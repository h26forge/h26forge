# Test vectors found at https://www.itu.int/net/ITU-T/sigdb/spevideo/VideoForm-s.aspx?val=102002641
# This should contain 135 videos.

echo "WARNING: This will download and unzip a total of 12.7 GB of data"

read -p "Are you sure? [Y to continue]" -n 1 -r
echo    # (optional) move to a new line
if [[ $REPLY =~ ^[Yy]$ ]]
then
    curl "https://www.itu.int/wftp3/Public/t/testsignal/SpeVideo/H264-1/v2016_02/ITU-T_H.264.1(2016-02)_AVCv1_bitstreams.zip" --output avcv1_bitstreams.zip

    #unzip avcv1_bitstreams.zip
    for f in $(find AVCv1/ -maxdepth 1 -name '*.zip');
    do
        unzip $f -d AVCv1/
    done

    mkdir -p vidz

    # Move all the video files to one folder

    for f in $(find AVCv1/ -name '*.264');
    do
        mv $f vidz/$(basename $f)
    done

    for f in $(find AVCv1/ -name '*.avc');
    do
        mv $f vidz/"$(basename $f .avc)".264
    done

    for f in $(find AVCv1/ -name '*.h264');
    do
        mv $f vidz/"$(basename $f .h264)".264
    done

    for f in $(find AVCv1/ -name '*.26l');
    do
        mv $f vidz/"$(basename $f .26l)".264
    done

    for f in $(find AVCv1/ -name '*.jsv');
    do
        mv $f vidz/"$(basename $f .jsv)".264
    done

    for f in $(find AVCv1/ -name '*.jvt');
    do
        mv $f vidz/"$(basename $f .jvt)".264
    done

    # Run the tester script on our folder.
    python3 end-to-end-tester.py --testdir vidz/

fi
